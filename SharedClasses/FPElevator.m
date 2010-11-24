//
//  FPElevator.m
//  IronJump
//
//  Created by Filip Kunc on 5/5/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPElevator.h"
#import "FPTexture.h"
#import "FPTextureArray.h"
#import "FPMath.h"
#import "FPPlayer.h"

#if TARGET_OS_MAC
FPTextureArray *elevatorTextures = nil;
#endif

@implementation FPElevator

@synthesize x, y, widthSegments, endX, endY, isVisible, textureIndex;

#if TARGET_OS_MAC
+ (FPTexture *)loadTextureIfNeeded
{
	if (!elevatorTextures)
	{
		elevatorTextures = [[FPTextureArray alloc] init];
		[elevatorTextures addTexture:@"vytah01.png"];
		[elevatorTextures addTexture:@"vytah02.png"];
		[elevatorTextures addTexture:@"vytah03.png"];		
	}
	return [elevatorTextures textureAtIndex:0];
}
#endif

- (id)init
{
	return [self initWithWidthSegments:1 endX:0.0f endY:96.0f];
}

- (id)initWithWidthSegments:(int)aWidthSegments endX:(float)anEndX endY:(float)anEndY
{
	self = [super init];
	if (self)
	{
		textureIndex = 0;
		animationCounter = 0;
		startX = x = 0.0f;
		startY = y = 0.0f;
		endX = anEndX;
		endY = anEndY;
		widthSegments = aWidthSegments;
		movingToEnd = YES;
		affectedObjects = nil;
        isVisible = YES;
	}
	return self;
}

- (void)dealloc
{
	[elevatorEnd release];
	[affectedObjects release];
	[super dealloc];
}

- (void)encodeWithCoder:(NSCoder *)aCoder
{
	[aCoder encodeFloat:x forKey:@"x"];
	[aCoder encodeFloat:y forKey:@"y"];
	[aCoder encodeInt:widthSegments forKey:@"widthSegments"];
	[aCoder encodeFloat:endX forKey:@"endX"];
	[aCoder encodeFloat:endY forKey:@"endY"];
}

- (id)initWithCoder:(NSCoder *)aDecoder
{
	self = [self initWithWidthSegments:1 endX:0.0f endY:0.0f];
	if (self)
	{
		startX = x = [aDecoder decodeFloatForKey:@"x"];
		startY = y = [aDecoder decodeFloatForKey:@"y"];
		widthSegments = [aDecoder decodeIntForKey:@"widthSegments"];
		endX = [aDecoder decodeFloatForKey:@"endX"];
		endY = [aDecoder decodeFloatForKey:@"endY"];
	}
	return self;
}

- (CGRect)rect
{
	return CGRectMake(x, y, widthSegments * 32.0f, 32.0f);
}

- (BOOL)isPlatform
{
	return YES;
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
	startX += offsetX;
	startY += offsetY;
	if (!elevatorEnd)
	{
		endX += offsetX;
		endY += offsetY;
	}
}

- (void)moveCurrentX:(float)offsetX y:(float)offsetY
{
	x += offsetX;
	y += offsetY;
}

- (void)elevatorCollision:(id<FPGameProtocol>)game diffX:(float)diffX diffY:(float)diffY
{
	FPPlayer *player = (FPPlayer *)[game player];
	CGRect playerRect = player.rect;
	CGRect moveRect = CGRectWithMove(self.rect, diffX, diffY);
	
	if (diffY > 0.0f)
	{
        if (CGRectIntersectsRectWithTolerance(playerRect, moveRect))
			diffY = 0.0f;
		else
		{
			for (id<FPGameObject> gameObject in game.gameObjects)
			{
				if (gameObject.isMovable)
				{
                    if (CGRectIntersectsRectWithTolerance(gameObject.rect, moveRect))
					{
						diffY = 0.0f;
						break;
					}
				}
			}
		}
	}
    
    if (fabsf(diffX) > 0.0f)
	{
        if (CGRectIntersectsRectWithTolerance(playerRect, moveRect))
			diffX = 0.0f;
		else
		{
			for (id<FPGameObject> gameObject in game.gameObjects)
			{
				if (gameObject.isMovable)
				{
                    if (CGRectIntersectsRectWithTolerance(gameObject.rect, moveRect))
					{
						diffX = 0.0f;
						break;
					}
				}
			}
		}
	}
		
	playerRect.size.height += tolerance;	

    if (CGRectIntersectsRectWithTolerance(playerRect, moveRect))
	{
		[game moveWorldWithX:-diffX y:0.0f];			
		[player collisionLeftRight:game];
        [game moveWorldWithX:0.0f y:-diffY];
 	}
	else
	{
		for (id<FPGameObject> gameObject in affectedObjects)
		{
			moveRect = CGRectWithMove(gameObject.rect, diffX, diffY);
  			if (CGRectIntersectsRectWithTolerance(playerRect, moveRect))
			{
				[game moveWorldWithX:-diffX y:0.0f];
				[player collisionLeftRight:game];
                [game moveWorldWithX:0.0f y:-diffY];
				break;
			}					
		}
	}
    
	id<FPGameObject> movableOnElevator = nil;
	CGRect movableRect;
	
	for (id<FPGameObject> movable in game.gameObjects)
	{
		if (movable.isMovable)
		{
			moveRect = CGRectWithMove(self.rect, diffX, diffY);			
			movableRect = movable.rect;
			movableRect.size.height += tolerance;
			if (CGRectGetMaxY(movableRect) < CGRectGetMaxY(moveRect) &&
				CGRectIntersectsRectWithTolerance(movableRect, moveRect))
			{
				movableOnElevator = movable;
				break;
			}
			else
			{
				for (id<FPGameObject> gameObject in affectedObjects)
				{
					moveRect = CGRectWithMove(gameObject.rect, diffX, diffY);
					if (CGRectGetMaxY(movableRect) < CGRectGetMaxY(moveRect) &&
						CGRectIntersectsRectWithTolerance(movableRect, moveRect))
					{
						movableOnElevator = movable;
						break;
					}
				}
			}
		}
	}
	
	if (movableOnElevator)
	{
		movableRect = CGRectWithMove(movableOnElevator.rect, diffX, diffY);
		[movableOnElevator moveWithX:diffX y:0.0f];		
		if (CGRectIntersectsRectWithTolerance(playerRect, movableRect))
		{
			[game moveWorldWithX:-diffX y:0.0f];
			[player collisionLeftRight:game];
        }
        [movableOnElevator moveWithX:0.0f y:diffY];		
		if (CGRectIntersectsRectWithTolerance(playerRect, movableRect))
		{
		    [game moveWorldWithX:0.0f y:-diffY];
		}
	}

	x += diffX;
	y += diffY;
	for (id<FPGameObject> gameObject in affectedObjects)
		[gameObject moveWithX:diffX y:diffY];
}

- (void)initAffectedObjectsIfNeeded:(id<FPGameProtocol>)game
{
	if (!affectedObjects)
	{
		affectedObjects = [[NSMutableArray alloc] init];
		CGRect selfRect = [self rect];
		for (id<FPGameObject> gameObject in game.gameObjects)
		{
			if (gameObject == self)
				continue;
			
			CGRect gameObjectRect = [gameObject rect];
			gameObjectRect.size.height += tolerance;
			if (CGRectIntersectsRect(gameObjectRect, selfRect))
				[affectedObjects addObject:gameObject];
		}
		
		for (id<FPGameObject> gameObject in game.gameObjects)
		{
			if (gameObject == self)
				continue;
			
			if ([affectedObjects containsObject:gameObject])
				continue;
			
			CGRect gameObjectRect = [gameObject rect];
			gameObjectRect.size.height += tolerance;
			
			for (id<FPGameObject> affectedObject in affectedObjects)
			{
				if (CGRectIntersectsRect(gameObjectRect, [affectedObject rect]))
					[affectedObjects addObject:gameObject];
			}
		}
	}
}

- (void)updateWithGame:(id<FPGameProtocol>)game
{
	const float speed = 2.0f;
	
	float diffX, diffY;
	
	if (movingToEnd)
	{
		diffX = endX - x;
		diffY = endY - y;
	}
	else
	{
		diffX = startX - x;
		diffY = startY - y;
	}
	
	diffX = fabsminf(diffX, speed);
	diffY = fabsminf(diffY, speed);
	
	if (fabsf(diffX) < 0.1f && fabsf(diffY) < 0.1f)
	{
		movingToEnd = !movingToEnd;
	}
	
	[self initAffectedObjectsIfNeeded:game];
	[self elevatorCollision:game diffX:diffX diffY:diffY];
	
	if (textureIndex > 2)
		textureIndex = 2;	
	
	if (textureIndex < 0)
		textureIndex = 0;
	
	if (diffY < 0.0f)
	{
		if (++animationCounter > 2)
		{
			animationCounter = 0;
			if (++textureIndex >= 2)
				textureIndex = 2;
		}
	}
	else if (diffY > 0.0f)
	{
		if (++animationCounter > 2)
		{
			animationCounter = 0;
			if (--textureIndex < 0)
				textureIndex = 0;
		}
	}
	else
	{
		textureIndex = 1;
	}
}

- (void)draw
{
#if TARGET_OS_IPHONE
	[[FPGameAtlas sharedAtlas] addElevator:textureIndex atPoint:CGPointMake(x, y) widthSegments:widthSegments heightSegments:1];
#else
	[FPElevator loadTextureIfNeeded];
	FPTexture *texture = [elevatorTextures textureAtIndex:textureIndex];
	[texture drawAtPoint:CGPointMake(x, y) widthSegments:widthSegments heightSegments:1];
#endif
}

- (id<FPGameObject>)duplicateWithOffsetX:(float)offsetX offsetY:(float)offsetY
{
	FPElevator *duplicated = [[[FPElevator alloc] initWithWidthSegments:widthSegments endX:endX - x endY:endY - y] autorelease];
	[duplicated moveWithX:x + offsetX y:y + offsetY];
	return duplicated;
}

- (id<FPGameObject>)nextPart
{
	if (elevatorEnd == nil)
		elevatorEnd = [[FPElevatorEnd alloc] initWithElevatorStart:self];
	return elevatorEnd;
}

- (void)parseXMLElement:(NSString *)elementName value:(NSString *)value
{
    if ([elementName isEqualToString:@"x"])
        startX = x = [value floatValue];
    else if ([elementName isEqualToString:@"y"])
        startY = y = [value floatValue];
    else if ([elementName isEqualToString:@"endX"])
        endX = [value floatValue];
    else if ([elementName isEqualToString:@"endY"])
        endY = [value floatValue];
    else if ([elementName isEqualToString:@"widthSegments"])
        widthSegments = [value floatValue];
}

- (void)writeToXML:(FPXMLWriter *)writer
{
    [writer writeElementWithName:@"x" floatValue:x];
    [writer writeElementWithName:@"y" floatValue:y];
    [writer writeElementWithName:@"endX" floatValue:endX];
    [writer writeElementWithName:@"endY" floatValue:endY];
    [writer writeElementWithName:@"widthSegments" intValue:widthSegments];
}

@end

@implementation FPElevatorEnd

- (id)initWithElevatorStart:(FPElevator *)anElevatorStart
{
	self = [super init];
	if (self)
	{
		elevatorStart = anElevatorStart;
	}
	return self;
}

- (float)x
{
	return elevatorStart.endX;
}

- (float)y
{
	return elevatorStart.endY;
}

- (int)widthSegments
{
	return elevatorStart.widthSegments;
}

- (void)setWidthSegments:(int)aWidthSegments
{
	elevatorStart.widthSegments = aWidthSegments;
}

- (CGRect)rect
{
	return CGRectMake(self.x, self.y, 
					  self.widthSegments * 32.0f, 32.0f);
}

- (BOOL)isVisible
{
	return YES;
}

- (void)setIsVisible:(BOOL)value
{
	
}

- (BOOL)isPlatform
{
	return YES;
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
	elevatorStart.endX += offsetX;
	elevatorStart.endY += offsetY;
}

- (void)draw
{
#if !TARGET_OS_IPHONE
	[FPElevator loadTextureIfNeeded];
	glColor4f(1, 1, 1, 0.5f);	
	FPTexture *texture = [elevatorTextures textureAtIndex:0];
	[texture drawAtPoint:CGPointMake(self.x, self.y) widthSegments:self.widthSegments heightSegments:1];	

	glDisable(GL_TEXTURE_2D);
	
	CGPoint start = CGRectMiddlePoint([elevatorStart rect]);
	CGPoint end = CGRectMiddlePoint([self rect]);
	

	glColor4f(0, 1, 0, 1);
	glBegin(GL_LINES);
	glVertex2f(start.x, start.y);
	glVertex2f(end.x, end.y);
	glEnd();
	
#endif
}

- (id<FPGameObject>)duplicateWithOffsetX:(float)offsetX offsetY:(float)offsetY
{
	return nil;
}

- (id<FPGameObject>)nextPart
{
	return elevatorStart;
}

@end

