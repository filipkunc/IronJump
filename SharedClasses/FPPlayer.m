//
//  FPPlayer.m
//  IronJump
//
//  Created by Filip Kunc on 5/2/10.
//  For license see LICENSE.TXT
//

#import "FPPlayer.h"
#import "FPTexture.h"
#import "FPTextureArray.h"

#if TARGET_OS_MAC
FPTexture *playerTexture = nil;
FPTexture *jumpTexture = nil;
#endif

const float tolerance = 3.0f;
const float maxSpeed = 5.8f;
const float speedPowerUp = 1.5f;
const float upSpeed = 7.0f;
const float maxFallSpeed = -15.0f;
const float acceleration = 1.1f;
const float deceleration = 1.1f * 0.2f;
const float changeDirectionSpeed = 3.0f;
const int maxSpeedUpCount = 60 * 6; // 60 FPS * 6 sec

@implementation FPPlayer

@synthesize x, y, moveX, moveY, speedUpCounter, isVisible, rotation, alpha;

#if TARGET_OS_MAC
+ (FPTexture *)loadTextureIfNeeded
{
	if (!playerTexture)
	{
		playerTexture = [[FPTexture alloc] initWithFile:@"ball.png" convertToAlpha:NO];
		jumpTexture = [[FPTexture alloc] initWithFile:@"speed.png" convertToAlpha:NO];
	}
	return playerTexture;
}
#endif

- (id)init
{
	self = [super init];
	if (self)
	{
		x = 0.0f;
		y = 0.0f;
		moveX = 0.0f;
		moveY = 0.0f;
		jumping = NO;
		speedUpCounter = 0;
		alpha = 1.0f;
        isVisible = YES;
	}
	return self;
}

- (id)initWithWidth:(float)aWidth height:(float)aHeight
{
	self = [self init];
	if (self)
	{
		x = aWidth / 2.0f - 32.0f / 2.0f;
		y = aHeight / 2.0f - 32.0f / 2.0f;
	}
	return self;
}

- (void)dealloc
{
	[super dealloc];
}

- (void)encodeWithCoder:(NSCoder *)aCoder
{
	[aCoder encodeFloat:x forKey:@"x"];
	[aCoder encodeFloat:y forKey:@"y"];
}

- (id)initWithCoder:(NSCoder *)aDecoder
{
	self = [self init];
	if (self)
	{
		x = [aDecoder decodeFloatForKey:@"x"];
		y = [aDecoder decodeFloatForKey:@"y"];
	}
	return self;
}

- (CGRect)rect
{
	return CGRectMake(x, y, 32.0f, 32.0f);
}

- (BOOL)isPlatform
{
	return NO;
}

- (BOOL)isMovable
{
	return NO;
}

- (BOOL)isTransparent
{
	return YES;
}

- (void)moveWithX:(float)offsetX y:(float)offsetY
{
	x += offsetX;
	y += offsetY;
}

- (void)updateWithGame:(id<FPGameProtocol>)game
{	
	CGPoint inputAcceleration = [game inputAcceleration];
	BOOL moveLeftOrRight = NO;
	
	if (speedUpCounter > 0)
	{
		if (++speedUpCounter > maxSpeedUpCount)
		{
			speedUpCounter = 0;			
		}
	}
	
	float currentMaxSpeed = speedUpCounter > 0 ? maxSpeed * speedPowerUp : maxSpeed;
	
	if (inputAcceleration.x < 0.0f)
	{
		if (moveX < 0.0f)
			moveX += fabsf(inputAcceleration.x) * acceleration * changeDirectionSpeed;
		moveX += fabsf(inputAcceleration.x) * acceleration;
		if (moveX > currentMaxSpeed)
			moveX = currentMaxSpeed;
		
		moveLeftOrRight = YES;
	}
	else if (inputAcceleration.x > 0.0f)
	{
		if (moveX > 0.0f)
			moveX -= fabsf(inputAcceleration.x) * acceleration * changeDirectionSpeed;
		moveX -= fabsf(inputAcceleration.x) * acceleration;
		if (moveX < -currentMaxSpeed)
			moveX = -currentMaxSpeed;
		
		moveLeftOrRight = YES;
	}
	
	if (!jumping && inputAcceleration.y > 0.0f)
	{
		if (moveY < upSpeed)
			moveY = upSpeed;
		jumping = YES;
	}
	
	if (!moveLeftOrRight)
	{
		if (fabsf(moveX) < deceleration)
			moveX = 0.0f;
		else if (moveX > 0.0f)
			moveX -= deceleration;
		else if (moveX < 0.0f)
			moveX += deceleration;
	}	
	
	moveY -= deceleration;
	if (moveY < maxFallSpeed)
		moveY = maxFallSpeed;
	jumping = YES;
	
	[game moveWorldWithX:moveX y:0.0f];
	if ([self collisionLeftRight:game])
		moveX = 0.0f;
    [game moveWorldWithX:0.0f y:moveY];
	[self collisionUpDown:game];
	rotation -= moveX * 3.0f;
	
	alpha += 0.07f;
	if (alpha > M_PI)
		alpha -= M_PI;
}

- (BOOL)collisionLeftRight:(id<FPGameProtocol>)game
{
	BOOL isColliding = NO;
	
	for (id<FPGameObject> platform in [game gameObjects])
	{
		if (platform.isPlatform)
		{
			CGRect intersection = CGRectIntersection(platform.rect, self.rect);
			if (CGRectIsEmptyWithTolerance(intersection))
				continue;
			
			if (CGRectGetMinX(platform.rect) > CGRectGetMinX(self.rect))
			{
                if (platform.isMovable)
                {
                    [platform moveWithX:intersection.size.width y:0.0f];
                    if ([platform collisionLeftRight:game])
                    {
                        [platform moveWithX:-intersection.size.width y:0.0f];
                        [game moveWorldWithX:intersection.size.width y:0.0f];
                        isColliding = YES;
                    }
                }
                else
                {
                    [game moveWorldWithX:intersection.size.width y:0.0f];
                    isColliding = YES;
                }					
			}
			else if (CGRectGetMaxX(platform.rect) < CGRectGetMaxX(self.rect))
			{
                if (platform.isMovable)
                {
                    [platform moveWithX:-intersection.size.width y:0.0f];
                    if ([platform collisionLeftRight:game])
                    {
                        [platform moveWithX:intersection.size.width y:0.0f];
                        [game moveWorldWithX:-intersection.size.width y:0.0f];
                        isColliding = YES;
                    }
                }
                else
                {
                    [game moveWorldWithX:-intersection.size.width y:0.0f];
                    isColliding = YES;
                }
            }
		}
	}
	
	return isColliding;
}

- (BOOL)collisionUpDown:(id<FPGameProtocol>)game
{
	BOOL isColliding = NO;
	
	for (id<FPGameObject> platform in [game gameObjects])
	{
		if (platform.isPlatform)
		{
			CGRect intersection = CGRectIntersection(platform.rect, self.rect);
			if (CGRectIsEmptyWithTolerance(intersection))
				continue;
			
			if (CGRectGetMaxY(platform.rect) < CGRectGetMaxY(self.rect))
			{
				if (moveY > 0.0f)
					moveY = 0.0f;

				[game moveWorldWithX:0.0f y:-intersection.size.height];
				isColliding = YES;
			}
			else if (moveY < 0.0f)
			{
				if (CGRectGetMinY(platform.rect) > CGRectGetMaxY(self.rect) - tolerance + moveY)
				{
					moveY = 0.0f;
					jumping = NO;
					[game moveWorldWithX:0.0f y:intersection.size.height];
					isColliding = YES;
				}
			}
			else if (CGRectGetMinY(platform.rect) > CGRectGetMaxY(self.rect) - tolerance + moveY)
			{
				jumping = NO;
				[game moveWorldWithX:0.0f y:intersection.size.height];
				isColliding = YES;
			}
		}
	}
	
	return isColliding;
}

- (void)draw
{
#if TARGET_OS_IPHONE
	[[FPGameAtlas sharedAtlas] addPlayerAtPoint:CGPointMake(x, y) rotation:rotation];
#else
	[FPPlayer loadTextureIfNeeded];
	glPushMatrix();
	glTranslatef(x, y, 0.0f);	
	glTranslatef(playerTexture.width / 2.0f, playerTexture.height / 2.0f, 0.0f);
	glRotatef(rotation, 0, 0, 1);	
	glTranslatef(-playerTexture.width / 2.0f, -playerTexture.height / 2.0f, 0.0f);
	[playerTexture draw];
	glPopMatrix();
#endif
}

- (void)drawSpeedUp
{
	if (speedUpCounter <= 0)
		return;	
	
	glColor4f(1, 1, 1, fabsf(sinf(alpha)) * 0.5f + 0.5f);
	CGPoint point = CGPointMake(x - 16.0f, y - 16.0f);
	
#if TARGET_OS_IPHONE
	[[FPGameAtlas sharedAtlas] addSpeedEffectAtPoint:point];
	[[FPGameAtlas sharedAtlas] drawAllTiles];
#else
	[jumpTexture drawAtPoint:point];
#endif
}

- (id<FPGameObject>)duplicateWithOffsetX:(float)offsetX offsetY:(float)offsetY
{
	FPPlayer *duplicated = [[[FPPlayer alloc] init] autorelease];
	[duplicated moveWithX:x + offsetX y:y + offsetY];
	return duplicated;
}

- (void)parseXMLElement:(NSString *)elementName value:(NSString *)value
{
    if ([elementName isEqualToString:@"x"])
        x = [value floatValue];
    else if ([elementName isEqualToString:@"y"])
        y = [value floatValue];
}

- (void)writeToXML:(FPXMLWriter *)writer
{
    [writer writeElementWithName:@"x" floatValue:x];
    [writer writeElementWithName:@"y" floatValue:y];
}


@end
