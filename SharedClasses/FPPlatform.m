//
//  FPPlatform.m
//  IronJump
//
//  Created by Filip Kunc on 5/2/10.
//  For license see LICENSE.TXT
//

#import "FPPlatform.h"

#if TARGET_OS_MAC
FPTexture *platformTexture = nil;
#endif

@implementation FPPlatform

@synthesize x, y, widthSegments, heightSegments, isVisible;

#if TARGET_OS_MAC
+ (FPTexture *)loadTextureIfNeeded
{
	if (!platformTexture)
		platformTexture = [[FPTexture alloc] initWithFile:@"plos_marble.png" convertToAlpha:NO];
	return platformTexture;
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
		x = 0;
		y = 0;	
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
	self = [self init];
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
	return NO;
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
	
}

- (void)draw
{
#if TARGET_OS_IPHONE
	[[FPGameAtlas sharedAtlas] addPlatformAtPoint:CGPointMake(x, y) widthSegments:widthSegments heightSegments:heightSegments];
#else
	[FPPlatform loadTextureIfNeeded];
	[platformTexture drawAtPoint:CGPointMake(x, y) widthSegments:widthSegments heightSegments:heightSegments];
#endif
}

- (id<FPGameObject>)duplicateWithOffsetX:(float)offsetX offsetY:(float)offsetY
{
	FPPlatform *duplicated = [[[FPPlatform alloc] initWithWidthSegments:widthSegments heightSegments:heightSegments] autorelease];
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
