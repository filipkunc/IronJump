//
//  FPMovablePlatform.m
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 6/27/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPMovablePlatform.h"
#import "FPPlayer.h"

#if TARGET_OS_MAC
FPTexture *movablePlatformTexture = nil;
#endif

@implementation FPMovablePlatform

@synthesize x, y, widthSegments, heightSegments, moveY, isVisible;

#if TARGET_OS_MAC
+ (FPTexture *)loadTextureIfNeeded
{
	if (!movablePlatformTexture)
		movablePlatformTexture = [[FPTexture alloc] initWithFile:@"movable.png" convertToAlpha:NO];
	return movablePlatformTexture;
}
#endif

- (id)init
{
	return [self initWithWidthSegments:1 heightSegments:1];
}

- (id)initWithWidthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments
{
	self = [super init];
	if (self)
	{
		x = 0.0f;
		y = 0.0f;	
		moveY = 0.0f;
		widthSegments = aWidthSegments;
		heightSegments = aHeightSegments;
        isVisible = YES;
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
	[aCoder encodeInt:widthSegments forKey:@"widthSegments"];
	[aCoder encodeInt:heightSegments forKey:@"heightSegments"];
}

- (id)initWithCoder:(NSCoder *)aDecoder
{
	self = [self initWithWidthSegments:1 heightSegments:1];
	if (self)
	{
		x = [aDecoder decodeFloatForKey:@"x"];
		y = [aDecoder decodeFloatForKey:@"y"];
		widthSegments = [aDecoder decodeIntForKey:@"widthSegments"];
		if ([aDecoder containsValueForKey:@"heightSegments"])
			heightSegments = [aDecoder decodeIntForKey:@"heightSegments"];
		else
			heightSegments = 1;
	}
	return self;
}

- (CGRect)rect
{
	return CGRectMake(x, y, 32.0f * widthSegments, 32.0f * heightSegments);
}

- (BOOL)isPlatform
{
	return YES;
}

- (BOOL)isMovable
{
	return YES;
}

- (BOOL)isTransparent
{
	return NO;
}

- (void)moveWithX:(float)offsetX y:(float)offsetY
{
	x += offsetX;
	y += offsetY;
}

- (void)updateWithGame:(id<FPGameProtocol>)game
{
	FPPlayer *player = (FPPlayer *)[game player];
	CGRect playerRect = player.rect;
	
	moveY -= deceleration;
	if (moveY < maxFallSpeed)
		moveY = maxFallSpeed;
	
	CGRect moveRect = CGRectWithMove(self.rect, 0.0f, -moveY);
	
    if (CGRectIntersectsRectWithTolerance(playerRect, moveRect))
		moveY = 0.0f;
	
	y -= moveY;
	[self collisionUpDown:game];
}

- (BOOL)collisionLeftRight:(id<FPGameProtocol>)game
{
	for (id<FPGameObject> platform in [game gameObjects])
	{
		if (platform != self && platform.isPlatform)
		{
			CGRect intersection = CGRectIntersection(platform.rect, self.rect);
			if (CGRectIsEmptyWithTolerance(intersection))
				continue;
			
			if (CGRectGetMinX(platform.rect) > CGRectGetMinX(self.rect))
			{
				return YES;
			}
			else if (CGRectGetMaxX(platform.rect) < CGRectGetMaxX(self.rect))
			{
				return YES;
			}
		}
	}
	
	return NO;
}

- (BOOL)collisionUpDown:(id<FPGameProtocol>)game
{
	BOOL isColliding = NO;
	
	for (id<FPGameObject> platform in [game gameObjects])
	{
		if (platform != self && platform.isPlatform)
		{
			CGRect intersection = CGRectIntersection(platform.rect, self.rect);
			if (CGRectIsEmptyWithTolerance(intersection))
				continue;
			
			if (CGRectGetMaxY(platform.rect) < CGRectGetMaxY(self.rect))
			{
				if (moveY > 0.0f)
					moveY = 0.0f;
                
                y += intersection.size.height;
				isColliding = YES;
			}
			else if (moveY < 0.0f)
			{
				if (CGRectGetMinY(platform.rect) > CGRectGetMaxY(self.rect) - tolerance + moveY)
				{
					moveY = 0.0f;
                    y -= intersection.size.height;
					isColliding = YES;
				}
			}
			else if (CGRectGetMinY(platform.rect) > CGRectGetMaxY(self.rect) - tolerance + moveY)
			{
                y -= intersection.size.height;
				isColliding = YES;
			}
		}
	}
	
	return isColliding;
}

- (void)draw
{
#if TARGET_OS_IPHONE
	[[FPGameAtlas sharedAtlas] addMovableAtPoint:CGPointMake(x, y) widthSegments:widthSegments heightSegments:heightSegments];
#else
	[FPMovablePlatform loadTextureIfNeeded];
	[movablePlatformTexture drawAtPoint:CGPointMake(x, y) widthSegments:widthSegments heightSegments:heightSegments];
#endif
}

- (id<FPGameObject>)duplicateWithOffsetX:(float)offsetX offsetY:(float)offsetY
{
	FPMovablePlatform *duplicated = [[[FPMovablePlatform alloc] initWithWidthSegments:widthSegments heightSegments:heightSegments] autorelease];
	[duplicated moveWithX:x + offsetX y:y + offsetY];
	return duplicated;
}

- (void)parseXMLElement:(NSString *)elementName value:(NSString *)value
{
    if ([elementName isEqualToString:@"x"])
        x = [value floatValue];
    else if ([elementName isEqualToString:@"y"])
        y = [value floatValue];
    else if ([elementName isEqualToString:@"widthSegments"])
        widthSegments = [value floatValue];
    else if ([elementName isEqualToString:@"heightSegments"])
        heightSegments = [value floatValue];
}

- (void)writeToXML:(FPXMLWriter *)writer
{
    [writer writeElementWithName:@"x" floatValue:x];
    [writer writeElementWithName:@"y" floatValue:y];
    [writer writeElementWithName:@"widthSegments" intValue:widthSegments];
    [writer writeElementWithName:@"heightSegments" intValue:heightSegments];
}

@end
